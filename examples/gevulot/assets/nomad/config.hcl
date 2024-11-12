data_dir = "/tmp/nomad/"
bind_addr = "0.0.0.0"

addresses {
  http = "127.0.0.1"
  serf = "0.0.0.0"
  rpc = "0.0.0.0"
}

ports {
  http = 9646
  rpc  = 9647
  serf = 9648
}

region = "local"
datacenter = "local"

server {
  enabled          = true
  bootstrap_expect = 1
  authoritative_region = "local"
}

leave_on_interrupt = true
leave_on_terminate = true

client {
  enabled = true
  max_kill_timeout = "30s"
  node_class = "local_node"
  # cgroup_parent = "/nomad"

  options {
    "docker.privileged.enabled" = "true"
    "docker.volumes.enabled" = "true"
  }
}

plugin "qemu" {
  config {
  }
}

plugin "raw_exec" {
  config {
    enabled = true
  }
}

telemetry {
  prometheus_metrics         = true
  publish_allocation_metrics = true
  publish_node_metrics       = true
  disable_hostname = true
  collection_interval = "10s"
}
