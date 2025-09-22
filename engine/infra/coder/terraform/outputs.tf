output "public_ip" { value = aws_instance.coder.public_ip }
output "public_dns" { value = aws_instance.coder.public_dns }
output "url" {
  value = var.domain_name != "" ? "https://${var.domain_name}" : "http://${aws_instance.coder.public_dns}"
}
