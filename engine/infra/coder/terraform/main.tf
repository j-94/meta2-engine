data "aws_vpc" "default" {
  default = true
}

data "aws_subnets" "default" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default.id]
  }
}

data "aws_ami" "ubuntu_jammy" {
  most_recent = true
  owners      = ["099720109477"] # Canonical
  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }
  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

resource "aws_security_group" "coder" {
  name        = "coder-sg"
  description = "Coder ingress"
  vpc_id      = data.aws_vpc.default.id

  ingress { from_port = 22, to_port = 22, protocol = "tcp", cidr_blocks = [var.ssh_cidr] }
  ingress { from_port = 80, to_port = 80, protocol = "tcp", cidr_blocks = ["0.0.0.0/0"] }
  ingress { from_port = 443, to_port = 443, protocol = "tcp", cidr_blocks = ["0.0.0.0/0"] }
  egress  { from_port = 0, to_port = 0, protocol = "-1", cidr_blocks = ["0.0.0.0/0"] }

  tags = merge({ Name = "coder-sg" }, var.tags)
}

resource "aws_instance" "coder" {
  ami                    = data.aws_ami.ubuntu_jammy.id
  instance_type          = var.instance_type
  subnet_id              = data.aws_subnets.default.ids[0]
  vpc_security_group_ids = [aws_security_group.coder.id]
  key_name               = var.key_name != "" ? var.key_name : null

  user_data = file("${path.module}/../scripts/coder_bootstrap.sh")

  tags = merge({ Name = "coder-host" }, var.tags)
}

