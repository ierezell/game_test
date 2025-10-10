"""
Gymnasium-compatible wrapper for the Yolo game environment.

This module provides a standard Gymnasium interface for training RL agents
in the Yolo multiplayer survival horror game.
"""

from typing import Any, Dict, Optional, Tuple, Union
import numpy as np
import gymnasium as gym
from gymnasium import spaces
from gymnasium.envs.registration import register

try:
    import yolo_env  # The Rust extension module
except ImportError:
    print("Warning: yolo_env extension not built. Run 'maturin develop' to build.")
    yolo_env = None


class YoloGymEnvironment(gym.Env):
    """
    Gymnasium-compatible environment for Yolo game RL training.
    
    This environment wraps the Rust-based Yolo game to provide a standard
    Gymnasium interface for reinforcement learning training.
    """
    
    metadata = {
        "render_modes": ["human", "rgb_array", "ansi"],
        "render_fps": 60,
    }
    
    def __init__(self, 
                 episode_length: int = 1000,
                 max_episodes: int = 1000,
                 render_mode: Optional[str] = None,
                 **kwargs):
        """
        Initialize the Yolo Gymnasium environment.
        
        Args:
            episode_length: Maximum steps per episode
            max_episodes: Maximum number of episodes
            render_mode: Rendering mode ("human", "rgb_array", "ansi", or None)
        """
        super().__init__()
        
        if yolo_env is None:
            raise ImportError(
                "yolo_env extension not available. "
                "Please build with 'maturin develop' first."
            )
        
        self.render_mode = render_mode
        self._yolo_env = yolo_env.create_yolo_env(episode_length, max_episodes)
        
        # Define action space
        # Actions: movement (3D), look_direction (2D), jump, sprint, fire, reload, switch_weapon
        self.action_space = spaces.Dict({
            "movement": spaces.Box(low=-1.0, high=1.0, shape=(3,), dtype=np.float32),
            "look_direction": spaces.Box(low=[-np.pi, -np.pi/2], high=[np.pi, np.pi/2], shape=(2,), dtype=np.float32),
            "jump": spaces.Discrete(2),
            "sprint": spaces.Discrete(2),
            "fire": spaces.Discrete(2),
            "reload": spaces.Discrete(2),
            "switch_weapon": spaces.Discrete(5),  # -1, 0, 1, 2, 3 -> mapped to 0, 1, 2, 3, 4
        })
        
        # Define observation space
        self.observation_space = spaces.Dict({
            "player_position": spaces.Box(low=-np.inf, high=np.inf, shape=(3,), dtype=np.float32),
            "player_health": spaces.Box(low=0.0, high=100.0, shape=(1,), dtype=np.float32),
            "player_stamina": spaces.Box(low=0.0, high=100.0, shape=(1,), dtype=np.float32),
            "current_weapon": spaces.Discrete(4),  # 0=Pistol, 1=Rifle, 2=Shotgun, 3=Sniper
            "ammo_count": spaces.Box(low=0, high=999, shape=(1,), dtype=np.int32),
            "nearby_enemies": spaces.Box(low=-np.inf, high=np.inf, shape=(10, 3), dtype=np.float32),  # Max 10 enemies
            "nearby_players": spaces.Box(low=-np.inf, high=np.inf, shape=(4, 3), dtype=np.float32),   # Max 4 players
            "game_time": spaces.Box(low=0.0, high=np.inf, shape=(1,), dtype=np.float32),
        })
        
        # Weapon name to index mapping
        self._weapon_map = {
            "Pistol": 0,
            "Rifle": 1, 
            "Shotgun": 2,
            "Sniper": 3,
        }
    
    def reset(self, 
              *, 
              seed: Optional[int] = None, 
              options: Optional[Dict[str, Any]] = None) -> Tuple[Dict[str, np.ndarray], Dict[str, Any]]:
        """Reset the environment for a new episode."""
        super().reset(seed=seed)
        
        if seed is not None:
            # Set random seed in the Rust environment if supported
            pass
        
        obs_rust, info_dict = self._yolo_env.reset(seed)
        observation = self._convert_observation(obs_rust)
        info = {"rust_info": info_dict}
        
        return observation, info
    
    def step(self, action: Dict[str, Union[np.ndarray, int]]) -> Tuple[
        Dict[str, np.ndarray], float, bool, bool, Dict[str, Any]
    ]:
        """Step the environment with the given action."""
        # Convert Gymnasium action to Rust action
        rust_action = self._convert_action(action)
        
        # Step the Rust environment
        step_result = self._yolo_env.step(rust_action)
        
        # Convert back to Gymnasium format
        observation = self._convert_observation(step_result.observation)
        reward = float(step_result.reward)
        terminated = bool(step_result.terminated)
        truncated = bool(step_result.truncated)
        info = {"rust_info": step_result.info}
        
        return observation, reward, terminated, truncated, info
    
    def render(self) -> Optional[Union[np.ndarray, str]]:
        """Render the environment."""
        if self.render_mode is None:
            return None
        
        # Call Rust render function
        render_data = self._yolo_env.render(self.render_mode)
        
        if self.render_mode == "ansi":
            # Return text representation
            obs = self._yolo_env.get_observation()
            return f"""
=== Yolo Game State ===
Position: ({obs.player_position[0]:.2f}, {obs.player_position[1]:.2f}, {obs.player_position[2]:.2f})
Health: {obs.player_health:.1f}/100
Stamina: {obs.player_stamina:.1f}/100
Weapon: {obs.current_weapon}
Ammo: {obs.ammo_count}
Enemies nearby: {len(obs.nearby_enemies)}
Game time: {obs.game_time:.1f}s
"""
        elif self.render_mode == "human":
            # For human mode, print to console
            print(self.render())
            return None
        elif self.render_mode == "rgb_array":
            # Return a placeholder RGB array (would be actual game frame in real implementation)
            return np.zeros((480, 640, 3), dtype=np.uint8)
        
        return None
    
    def close(self):
        """Close the environment."""
        if hasattr(self, '_yolo_env'):
            self._yolo_env.close()
    
    def configure_rewards(self,
                         survival_reward: Optional[float] = None,
                         movement_reward_scale: Optional[float] = None,
                         kill_reward: Optional[float] = None,
                         damage_penalty_scale: Optional[float] = None,
                         death_penalty: Optional[float] = None):
        """Configure reward parameters."""
        self._yolo_env.set_rewards(
            survival_reward, movement_reward_scale, kill_reward, 
            damage_penalty_scale, death_penalty
        )
    
    def _convert_observation(self, rust_obs) -> Dict[str, np.ndarray]:
        """Convert Rust observation to Gymnasium observation."""
        # Pad enemy/player lists to fixed size
        enemies = np.array(rust_obs.nearby_enemies + [(0.0, 0.0, 0.0)] * (10 - len(rust_obs.nearby_enemies)))[:10]
        players = np.array(rust_obs.nearby_players + [(0.0, 0.0, 0.0)] * (4 - len(rust_obs.nearby_players)))[:4]
        
        return {
            "player_position": np.array(rust_obs.player_position, dtype=np.float32),
            "player_health": np.array([rust_obs.player_health], dtype=np.float32),
            "player_stamina": np.array([rust_obs.player_stamina], dtype=np.float32),
            "current_weapon": self._weapon_map.get(rust_obs.current_weapon, 0),
            "ammo_count": np.array([rust_obs.ammo_count], dtype=np.int32),
            "nearby_enemies": enemies.astype(np.float32),
            "nearby_players": players.astype(np.float32),
            "game_time": np.array([rust_obs.game_time], dtype=np.float32),
        }
    
    def _convert_action(self, gym_action: Dict[str, Union[np.ndarray, int]]):
        """Convert Gymnasium action to Rust action."""
        if yolo_env is None:
            raise RuntimeError("yolo_env not available")
            
        rust_action = yolo_env.Action()
        
        # Convert numpy arrays to tuples
        movement = tuple(gym_action["movement"].astype(float))
        look_direction = tuple(gym_action["look_direction"].astype(float))
        
        rust_action.movement = movement
        rust_action.look_direction = look_direction
        rust_action.jump = bool(gym_action["jump"])
        rust_action.sprint = bool(gym_action["sprint"])
        rust_action.fire = bool(gym_action["fire"])
        rust_action.reload = bool(gym_action["reload"])
        
        # Convert switch_weapon from 0-4 to -1-3
        switch_weapon_mapped = int(gym_action["switch_weapon"]) - 1
        rust_action.switch_weapon = switch_weapon_mapped
        
        return rust_action


# Register the environment with Gymnasium
register(
    id="YoloGame-v0",
    entry_point="yolo_gym_env:YoloGymEnvironment",
    max_episode_steps=1000,
    reward_threshold=None,
    kwargs={
        "episode_length": 1000,
        "max_episodes": 1000,
    }
)

# Register variants with different configurations
register(
    id="YoloGame-Short-v0",
    entry_point="yolo_gym_env:YoloGymEnvironment",
    max_episode_steps=300,
    kwargs={
        "episode_length": 300,
        "max_episodes": 1000,
    }
)

register(
    id="YoloGame-Long-v0", 
    entry_point="yolo_gym_env:YoloGymEnvironment",
    max_episode_steps=3000,
    kwargs={
        "episode_length": 3000,
        "max_episodes": 500,
    }
)


if __name__ == "__main__":
    # Simple test of the environment
    import gymnasium as gym
    
    # Create environment
    env = gym.make("YoloGame-v0", render_mode="ansi")
    
    print("Action space:", env.action_space)
    print("Observation space:", env.observation_space)
    
    # Test episode
    obs, info = env.reset(seed=42)
    print("Initial observation keys:", obs.keys())
    
    for step in range(10):
        # Sample random action
        action = env.action_space.sample()
        obs, reward, terminated, truncated, info = env.step(action)
        
        print(f"Step {step}: reward={reward:.3f}, terminated={terminated}, truncated={truncated}")
        
        if terminated or truncated:
            obs, info = env.reset()
            print("Episode finished, reset environment")
    
    env.close()
    print("Environment test completed!")