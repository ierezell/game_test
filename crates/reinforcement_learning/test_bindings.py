"""
Test script for Yolo environment Python bindings.

This script tests the basic functionality of the Gymnasium interface
without requiring the full Rust extension to be built.
"""

import sys
import traceback
from typing import Dict, Any
import numpy as np

def test_rust_bindings():
    """Test if Rust bindings can be imported and used."""
    print("\nTesting Rust bindings import...")
    
    try:
        import yolo_env
        print("✅ Rust yolo_env module imported successfully")
        
        # Test creating environment
        env = yolo_env.create_yolo_env(100, 10)
        print("✅ Rust environment created successfully")
        
        # Test action and observation objects
        action = yolo_env.Action()
        action.movement = (0.5, 0.0, 0.2)
        action.fire = True
        print("✅ Rust Action object created successfully")
        
        obs = yolo_env.Observation()
        obs.player_health = 95.5
        print("✅ Rust Observation object created successfully")
        
        return True
        
    except ImportError as e:
        print(f"ℹ️  Rust bindings not available: {e}")
        print("   This is expected if maturin develop hasn't been run yet")
        return False
    except Exception as e:
        print(f"❌ Error testing Rust bindings: {e}")
        traceback.print_exc()
        return False


def test_gymnasium_wrapper():
    """Test the Gymnasium wrapper if available."""
    print("\nTesting Gymnasium wrapper...")
    
    try:
        import yolo_gym_env
        print("✅ Gymnasium wrapper imported successfully")
        
        # Try to create environment (may fail if Rust bindings not built)
        try:
            import gymnasium as gym
            env = gym.make("YoloGame-v0")
            print("✅ Gymnasium environment created successfully")
            
            print(f"   Action space: {env.action_space}")
            print(f"   Observation space keys: {list(env.observation_space.spaces.keys())}")
            
            env.close()
            return True
            
        except Exception as e:
            print(f"ℹ️  Could not create full environment: {e}")
            print("   This is expected if Rust bindings are not built")
            return False
            
    except ImportError as e:
        print(f"❌ Could not import Gymnasium wrapper: {e}")
        return False


def main():
    """Run all tests."""
    print("🎮 Yolo Environment Python Bindings Test Suite")
    print("=" * 50)
    
    tests = [
        ("Action/Observation Spaces", test_action_observation_spaces),
        ("Mock Environment", test_mock_environment),
        ("Rust Bindings", test_rust_bindings),
        ("Gymnasium Wrapper", test_gymnasium_wrapper),
    ]
    
    results = []
    for name, test_func in tests:
        print(f"\n📋 Running: {name}")
        try:
            success = test_func()
            results.append((name, success))
        except Exception as e:
            print(f"❌ Test {name} failed with exception: {e}")
            results.append((name, False))
    
    print("\n" + "=" * 50)
    print("📊 Test Results Summary:")
    
    passed = 0
    for name, success in results:
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"   {status}: {name}")
        if success:
            passed += 1
    
    print(f"\n🎯 Total: {passed}/{len(results)} tests passed")
    
    if passed == len(results):
        print("🎉 All tests passed!")
        return 0
    else:
        print("⚠️  Some tests failed - see above for details")
        print("\nTo build Rust bindings:")
        print("   cd crates/python_bindings")
        print("   pip install maturin")
        print("   maturin develop --release")
        return 1


if __name__ == "__main__":
    sys.exit(main())