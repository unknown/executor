acl = "write"

key_prefix "vault/" {
    policy = "write"
}

service_prefix "" {
    policy = "write"
}

agent_prefix "" {
    policy = "read"
}

session_prefix "" {
    policy = "write"
}
