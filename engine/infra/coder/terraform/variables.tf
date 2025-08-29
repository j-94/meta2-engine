variable "region" { type = string }
variable "instance_type" { type = string, default = "t3.large" }
variable "key_name" { type = string, default = "" }
variable "ssh_cidr" { type = string, default = "0.0.0.0/0" }
variable "tags" { type = map(string), default = {} }

