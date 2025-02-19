# Otimizador de Imagens em Rust

Este projeto implementa uma solução altamente otimizada para **redimensionamento** e **conversão de formatos de imagens**, utilizando **Rust** para maximizar a eficiência. A arquitetura é baseada nos serviços da **AWS**, combinando **CloudFront Functions**, **AWS Lambda** e **Amazon S3** para fornecer um sistema escalável e de alto desempenho.

## Arquitetura

1. **Reescrita de URL com CloudFront Functions**  
   - Modifica as URLs das imagens solicitadas para apontar para versões otimizadas no S3 ou acionar o Lambda caso a imagem ainda não tenha sido processada.

2. **Processamento de Imagens com AWS Lambda (Rust)**  
   - Redimensiona e converte imagens sob demanda.
   - Suporta múltiplos formatos, incluindo **WebP** e **AVIF**, garantindo melhor compactação sem perda significativa de qualidade.

3. **Armazenamento no Amazon S3**  
   - As imagens processadas são armazenadas no **S3** para reutilização, reduzindo custos e melhorando o tempo de resposta.

## Benefícios

- 🚀 **Alto desempenho**: Rust proporciona um processamento rápido e eficiente.
- 📦 **Menor custo**: O uso de Lambda reduz custos operacionais, já que o processamento ocorre sob demanda.
- 🌍 **Distribuição global**: CloudFront melhora a entrega de imagens otimizadas com baixa latência.
- 🔄 **Conversão otimizada**: Suporte para **WebP** e **AVIF**, garantindo arquivos menores sem perder qualidade.

## Tecnologias Utilizadas

- **Rust**: Linguagem escolhida pelo alto desempenho e segurança.
- **AWS CloudFront Functions**: Reescrita dinâmica de URLs.
- **AWS Lambda**: Processamento serverless com Rust.
- **Amazon S3**: Armazenamento escalável de imagens otimizadas.
