"""
Example usage of Yolo Environment Python Bindings

This example demonstrates how to use the Yolo game environment
for reinforcement learning training once the bindings are built.

To build the Python extension:
1. Install Python 3.8+ and pip
2. Install maturin: pip install maturin
3. Build extension: maturin develop --release
4. Install dependencies: pip install gymnasium numpy

Then run this example:
python training_example.py
"""

import sys
import time
import random
from typing import Dict, Any, Tuple
import numpy as np

# These imports would work once the bindings are built
try:
    import gymnasium as gym
    import yolo_gym_env  # Registers the environments
    BINDINGS_AVAILABLE = True
except ImportError:
    print("Warning: Gymnasium and/or yolo_gym_env not available")
    print("Install with: pip install gymnasium numpy")
    print("Build bindings with: maturin develop --release")
    BINDINGS_AVAILABLE = False


def random_policy_example():
    """Example of random policy in Yolo environment."""
    if not BINDINGS_AVAILABLE:
        print("Skipping random policy example - bindings not available")
        return

    print("🎮 Running Random Policy Example")
    print("-" * 40)
    
    # Create environment
    env = gym.make("YoloGame-Short-v0", render_mode="ansi")
    
    # Configure rewards for exploration
    env.configure_rewards(
        survival_reward=0.1,
        movement_reward_scale=0.02,
        kill_reward=5.0,
        damage_penalty_scale=0.05,
        death_penalty=-5.0
    )
    
    episodes = 3
    
    for episode in range(episodes):
        print(f"\n🎯 Episode {episode + 1}")
        
        obs, info = env.reset(seed=42 + episode)
        episode_reward = 0.0
        steps = 0
        
        while True:
            # Random action
            action = env.action_space.sample()
            
            # Take step
            obs, reward, terminated, truncated, info = env.step(action)
            episode_reward += reward
            steps += 1
            
            # Print progress every 50 steps
            if steps % 50 == 0:
                health = obs["player_health"][0]
                stamina = obs["player_stamina"][0] 
                enemies = len([e for e in obs["nearby_enemies"] if np.any(e != 0)])
                print(f"  Step {steps}: Health={health:.1f}, Stamina={stamina:.1f}, Enemies={enemies}")
            
            if terminated or truncated:
                break
        
        # Episode summary
        final_health = obs["player_health"][0]
        game_time = obs["game_time"][0]
        rust_info = info.get("rust_info", {})
        
        print(f"✅ Episode {episode + 1} completed:")
        print(f"   Steps: {steps}")
        print(f"   Total Reward: {episode_reward:.2f}")
        print(f"   Final Health: {final_health:.1f}")
        print(f"   Game Time: {game_time:.1f}s")
        print(f"   Terminated: {terminated}, Truncated: {truncated}")
        
        if rust_info:
            kills = rust_info.get("kills", 0)
            distance = rust_info.get("total_distance", 0)
            print(f"   Kills: {kills}, Distance: {distance:.1f}")
    
    env.close()
    print("\n🎉 Random policy example completed!")


def simple_heuristic_example():
    """Example of simple heuristic policy."""
    if not BINDINGS_AVAILABLE:
        print("Skipping heuristic example - bindings not available")
        return

    print("\n🧠 Running Simple Heuristic Example")
    print("-" * 40)
    
    env = gym.make("YoloGame-v0", render_mode="ansi")
    
    def heuristic_action(obs: Dict[str, np.ndarray]) -> Dict[str, Any]:
        """Simple heuristic: move toward enemies if healthy, away if damaged."""
        action = {
            "movement": np.array([0.0, 0.0, 0.0], dtype=np.float32),
            "look_direction": np.array([0.0, 0.0], dtype=np.float32),
            "jump": False,
            "sprint": False,
            "fire": False,
            "reload": False,
            "switch_weapon": 0,  # No weapon switch
        }
        
        health = obs["player_health"][0]
        stamina = obs["player_stamina"][0]
        player_pos = obs["player_position"]
        
        # Find nearest enemy
        enemies = obs["nearby_enemies"]
        nearest_enemy = None
        min_distance = float('inf')
        
        for enemy_pos in enemies:
            if np.any(enemy_pos != 0):  # Non-zero position means enemy exists
                distance = np.linalg.norm(enemy_pos - player_pos)
                if distance < min_distance:
                    min_distance = distance
                    nearest_enemy = enemy_pos
        
        if nearest_enemy is not None:
            direction = nearest_enemy - player_pos
            direction_norm = np.linalg.norm(direction)
            
            if direction_norm > 0:
                direction = direction / direction_norm
                
                if health > 50:
                    # Move toward enemy if healthy
                    action["movement"] = direction.astype(np.float32)
                    action["fire"] = min_distance < 5.0  # Fire if close
                    action["sprint"] = stamina > 20
                else:
                    # Move away from enemy if damaged
                    action["movement"] = (-direction).astype(np.float32)
                    action["sprint"] = stamina > 10
                
                # Look at enemy
                yaw = np.arctan2(direction[1], direction[0])
                action["look_direction"] = np.array([yaw, 0.0], dtype=np.float32)
        else:
            # No enemies - explore randomly
            action["movement"] = np.random.uniform(-0.5, 0.5, 3).astype(np.float32)
            action["look_direction"] = np.random.uniform([-0.1, -0.1], [0.1, 0.1], 2).astype(np.float32)
        
        return action
    
    # Run heuristic policy
    obs, info = env.reset(seed=123)
    episode_reward = 0.0
    steps = 0
    
    while steps < 200:  # Run for 200 steps
        action = heuristic_action(obs)
        obs, reward, terminated, truncated, info = env.step(action)
        episode_reward += reward
        steps += 1
        
        if steps % 25 == 0:
            health = obs["player_health"][0]
            enemies = len([e for e in obs["nearby_enemies"] if np.any(e != 0)])
            print(f"  Heuristic Step {steps}: Health={health:.1f}, Enemies={enemies}, Reward={reward:.3f}")
        
        if terminated or truncated:
            break
    
    print(f"🧠 Heuristic policy completed: {episode_reward:.2f} total reward in {steps} steps")
    env.close()

def main():
    """Run all examples."""
    print("🎮 Yolo Environment Python Bindings Examples")
    print("=" * 50)
    
    if not BINDINGS_AVAILABLE:
        print("\n⚠️  Python bindings not built yet!")
        print("\nTo build the bindings:")
        print("1. Install Python 3.8+ and pip")
        print("2. Install maturin: pip install maturin")
        print("3. Navigate to: cd crates/python_bindings")
        print("4. Build extension: maturin develop --release")
        print("5. Install dependencies: pip install gymnasium numpy")
        print("6. Run this example again")
        print("\n" + "=" * 50)
    
    # Run examples
    try:
        random_policy_example()
        simple_heuristic_example()
        mock_training_example()
        
        print("\n🎉 All examples completed successfully!")
        
        if BINDINGS_AVAILABLE:
            print("\n✅ Python bindings are working correctly!")
        else:
            print("\n📝 Examples showed what's possible once bindings are built")
            
    except Exception as e:
        print(f"\n❌ Error running examples: {e}")
        import traceback
        traceback.print_exc()
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())