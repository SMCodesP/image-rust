# Serviço de Otimização de Imagens

Este serviço utiliza AWS Lambda, S3 e CloudFront para otimizar e servir imagens sob demanda.

## Pré-requisitos

- Rust (última versão estável)
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda)
- [Terraform](https://www.terraform.io/) (v1.0+)
- AWS CLI configurada com SSO
- Um bucket S3 existente com as imagens originais

## Configuração do Ambiente

1. Configure o AWS SSO:
```bash
aws sso login
```

2. Instale as dependências do Rust:
```bash
cargo build
```

## Deploy

O processo de deploy é dividido em duas partes: Lambda e Infraestrutura.

### 1. Deploy da Lambda

```bash
./deploy.sh
```

Este script vai:
- Compilar o código Rust para ARM64
- Fazer o deploy da função Lambda

### 2. Deploy da Infraestrutura (Terraform)

1. Entre no diretório do Terraform:
```bash
cd terraform
```

2. Copie o arquivo de exemplo de variáveis:
```bash
cp terraform.tfvars.example terraform.tfvars
```

3. Configure as variáveis em `terraform.tfvars`:
```hcl
aws_region = "sa-east-1"                      # Região AWS
aws_profile = "default"                       # Perfil AWS CLI
project_name = "img-optimization"             # Nome do projeto
destination_bucket_name = "bucket-cache"      # Bucket para cache (será criado)
lambda_function_name = "image_rust"           # Nome da função Lambda
```

4. Inicialize o Terraform:
```bash
terraform init
```

5. Verifique o plano de execução:
```bash
terraform plan
```

6. Aplique as mudanças:
```bash
terraform apply
```

**Nota**: A criação da distribuição CloudFront pode levar até 30 minutos.

## Arquitetura

- **Lambda Function**: Processa e otimiza as imagens
- **S3 Bucket de Origem**: Armazena as imagens originais (bucket existente)
- **S3 Bucket de Cache**: Armazena as imagens processadas
- **CloudFront**: CDN que serve as imagens e gerencia o cache

## Fluxo de Funcionamento

1. Usuário solicita uma imagem via CloudFront
2. CloudFront verifica se a imagem existe no bucket de cache
3. Se não existir:
   - Redireciona para a Lambda
   - Lambda busca a imagem original
   - Processa a imagem
   - Salva no bucket de cache
4. CloudFront serve a imagem otimizada

## Limpeza

Para remover toda a infraestrutura:

```bash
terraform destroy
```

**Nota**: Isso não vai remover:
- A função Lambda (gerenciada separadamente)
- O bucket S3 de origem (existente previamente)

## Variáveis de Ambiente

### Lambda
- `SOURCE_BUCKET`: Nome do bucket com as imagens originais
- `DESTINATION_BUCKET`: Nome do bucket de cache (definido no Terraform)

### Terraform
- `aws_region`: Região AWS
- `aws_profile`: Perfil AWS CLI
- `project_name`: Nome do projeto
- `destination_bucket_name`: Nome do bucket de cache
- `lambda_function_name`: Nome da função Lambda

## Troubleshooting

1. **Erro de Credenciais**: Execute `aws sso login --profile brscans`
2. **Erro no Deploy da Lambda**: Verifique as permissões do IAM
3. **CloudFront 403**: Verifique as políticas do bucket e OAI
