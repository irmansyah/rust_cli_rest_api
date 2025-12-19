# Project tracker
Simple rest api in rust

```bash
cargo run -- \
  --file "{{PROJECT_NAME}}/request_file/users/_user_config.json" \
  --tag user_get_all

```

## Config
put this in "{{PROJECT_PATH}}/request_file/users/_user_config.json"
```json
{
  "app_title": "This is {{PROJECT_NAME}} Users",
  "base_url": "http://localhost:8080/users",
  "variable_dir": "{{PROJECT_PATH}}/request_file/_variables",
  "variable_access_token_file": "ACCESS_TOKEN.txt",
  "headers": {
    "Content-Type": "application/json"
  },
  "requests": [
    {
      "req_tag": "user_register_customer",
      "req_title": "User Register Customer",
      "req_type": "POST",
      "req_end_point": "/register",
      "req_body": {
        "body_type": "RAW",
        "body_file": "{{PROJECT_PATH}}/request_file/users/user_post_register_customer_body.json"
      }
    },
    {
      "req_tag": "user_login_customer",
      "req_title": "User Login Customer",
      "req_type": "POST",
      "req_end_point": "/login",
      "req_variable_is_save": true,
      "req_variable_response_value": {
        "REFRESH_TOKEN.txt": "data.refresh_token",
        "ACCESS_TOKEN.txt": "data.access_token"
      },
      "req_body": {
        "body_type": "RAW",
        "body_file": "{{PROJECT_PATH}}/request_file/users/user_post_login_customer_body.json"
      }
    },
    {
      "req_tag": "user_refresh",
      "req_title": "Users Refresh",
      "req_type": "POST",
      "req_end_point": "/refresh",
      "req_variable_is_save": true,
      "req_variable_response_value": {
        "ACCESS_TOKEN.txt": "data.access_token",
        "REFRESH_TOKEN.txt": "data.refresh_token"
      },
      "req_body": {
        "body_type": "RAW",
        "body_file": "{{PROJECT_PATH}}/request_file/users/user_refresh_body.json"
      }
    },
    {
      "req_tag": "user_update_by_id",
      "req_title": "User Update by Id",
      "req_type": "PATCH",
      "req_end_point": "/one",
      "req_variable_type": "Bearer",
      "req_body": {
        "body_type": "RAW",
        "body_file": "{{PROJECT_PATH}}/request_file/users/user_patch_body.json"
      }
    },
    {
      "req_tag": "user_get_all",
      "req_title": "Get Users All",
      "req_type": "GET",
      "req_end_point": "/all",
      "req_variable_type": "Bearer",
      "req_body": null
    },
    {
      "req_tag": "user_delete_by_id",
      "req_title": "Delete Users by Id",
      "req_type": "DELETE",
      "req_end_point": "/one",
      "req_variable_type": "Bearer",
      "req_body": {
        "body_type": "RAW",
        "body_file": "{{PROJECT_PATH}}/request_file/users/user_delete_body.json"
      }
    }
  ]
}
```
