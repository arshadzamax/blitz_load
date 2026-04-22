# scripts/test_script.py
import random

def get_user_data():
    """Returns a dictionary to Rust"""
    users = ["Alice", "Bob", "Charlie", "Arshad"]
    return {
        "username": random.choice(users),
        "user_id": random.randint(1000, 9999),
        "is_active": True
    }