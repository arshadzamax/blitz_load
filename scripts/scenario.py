import uuid
import random

def get_payload():
    # Generate a unique identity
    unique_id = str(uuid.uuid4())[:8]
    test_password = f"Pass_{unique_id}!123"
    
    return {
        "name": f"User_{unique_id}",
        "email": f"loadtest_{unique_id}@example.com",
        "password": test_password,
        "confirmPassword": test_password  # Must match password for your API to succeed
    }