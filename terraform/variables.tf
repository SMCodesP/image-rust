variable "aws_region" {
  description = "Região AWS onde os recursos serão criados"
  type        = string
  default     = "sa-east-1"
}

variable "aws_profile" {
  description = "Perfil AWS a ser usado"
  type        = string
  default     = "default"
}

variable "project_name" {
  description = "Nome do projeto"
  type        = string
}

variable "destination_bucket_name" {
  description = "Nome do bucket S3 de destino (será criado)"
  type        = string
}

variable "lambda_function_name" {
  description = "Nome da função Lambda (sem o sufixo .lambda-url...)"
  type        = string
  default     = "image-rust"
}

variable "lambda_zip_path" {
  description = "Caminho para o arquivo ZIP da função Lambda"
  type        = string
  default     = "../target/lambda/image_rust/bootstrap.zip"
} 