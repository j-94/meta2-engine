# Coder on AWS (Minimal, PR-Driven)

This folder contains a minimal Terraform configuration and GitHub Action to provision a single EC2 host and run Coder via Docker.

## Prereqs
- GitHub OIDC → AWS IAM role with `sts:AssumeRole` for this repo
  - Save role ARN in repo secret `AWS_ROLE_ARN`
- Region secret `AWS_REGION`
- (optional) Provide `key_name` via TF var if you want SSH; otherwise use SSM or console

Tracking: open a new Issue using the template “Coder OIDC/AWS Setup”. When finished, edit the infra PR description to include `Closes #<issue_number>` so it auto‑closes on merge.

### Least‑Privilege Policy (example)
Start with EC2FullAccess to bootstrap, then swap to a scoped policy like `infra/coder/terraform/policies/least_privilege_ec2.json` attached to the OIDC role.

Apply via console (create a new policy with that JSON) or CLI, then detach EC2FullAccess.

## Run from GitHub
- Go to Actions → "Coder Deploy" → Run workflow
  - action: apply (to create) or destroy
  - region: (optional; defaults to AWS_REGION secret)
- Output: public IP/DNS of the EC2 instance; Coder listens on port 80

## Local Terraform
```bash
cd infra/coder/terraform
terraform init
terraform apply -var region=eu-west-1 -var ssh_cidr=$(curl -s ifconfig.me)/32
```

## Customize
- TLS/Domain: place behind ALB/CloudFront or run Caddy/Nginx for HTTPS
- SG: tighten `ssh_cidr` to your IP(s)
- Instance type: set `instance_type`

## Bootstrap Script
- `infra/coder/scripts/coder_bootstrap.sh` installs Docker and runs `ghcr.io/coder/coder` on port 80.

## Next
- Login to Coder: `http://<public-ip>/` and complete setup
- (Optional) Seed templates with `coder` CLI after first admin configured
