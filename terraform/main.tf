terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  profile = var.aws_profile
  region  = var.aws_region
}

# S3 Bucket de destino (cache)
resource "aws_s3_bucket" "destination" {
  bucket = var.destination_bucket_name
}

resource "aws_s3_bucket_public_access_block" "destination" {
  bucket = aws_s3_bucket.destination.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Origin Access Control para S3
resource "aws_cloudfront_origin_access_control" "s3_oac" {
  name                              = "${var.project_name}-s3-oac"
  description                       = "Origin Access Control para S3"
  origin_access_control_origin_type = "s3"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# Origin Access Control para Lambda
resource "aws_cloudfront_origin_access_control" "lambda_oac" {
  name                              = "${var.project_name}-lambda-oac"
  description                       = "Origin Access Control para Lambda"
  origin_access_control_origin_type = "lambda"
  signing_behavior                  = "always"
  signing_protocol                  = "sigv4"
}

# CloudFront Function para rewrite de URL
resource "aws_cloudfront_function" "url_rewrite" {
  name    = "${var.project_name}-url-rewrite"
  runtime = "cloudfront-js-1.0"
  code    = file("${path.module}/functions/url-rewrite.js")
}

# CloudFront Distribution
resource "aws_cloudfront_distribution" "main" {
  enabled = true
  
  # Origem S3 (cache)
  origin {
    domain_name              = aws_s3_bucket.destination.bucket_regional_domain_name
    origin_id                = "S3-${aws_s3_bucket.destination.id}"
    origin_access_control_id = aws_cloudfront_origin_access_control.s3_oac.id
  }

  # Origem Lambda
  origin {
    domain_name = "${var.lambda_function_name}.lambda-url.${var.aws_region}.on.aws"
    origin_id   = "LAMBDA-${aws_s3_bucket.destination.id}"
    origin_access_control_id = aws_cloudfront_origin_access_control.lambda_oac.id
    
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }

  # Origin Group (S3 + Lambda fallback)
  origin_group {
    origin_id = "GROUP-${aws_s3_bucket.destination.id}"

    failover_criteria {
      status_codes = [403, 404, 500, 503, 504]
    }

    member {
      origin_id = "S3-${aws_s3_bucket.destination.id}"
    }

    member {
      origin_id = "LAMBDA-${aws_s3_bucket.destination.id}"
    }
  }

  default_cache_behavior {
    allowed_methods        = ["GET", "HEAD", "OPTIONS"]
    cached_methods         = ["GET", "HEAD"]
    target_origin_id       = "GROUP-${aws_s3_bucket.destination.id}"
    viewer_protocol_policy = "redirect-to-https"
    
    function_association {
      event_type   = "viewer-request"
      function_arn = aws_cloudfront_function.url_rewrite.arn
    }

    forwarded_values {
      query_string = true
      cookies {
        forward = "none"
      }
    }

    min_ttl                = 0
    default_ttl            = 3600
    max_ttl                = 86400
    compress               = true
  }

  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }

  viewer_certificate {
    cloudfront_default_certificate = true
  }

  price_class = "PriceClass_200"
}

# Política de bucket para permitir acesso do CloudFront via OAC
resource "aws_s3_bucket_policy" "destination" {
  bucket = aws_s3_bucket.destination.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid       = "AllowCloudFrontServicePrincipal"
        Effect    = "Allow"
        Principal = {
          Service = "cloudfront.amazonaws.com"
        }
        Action   = "s3:GetObject"
        Resource = "${aws_s3_bucket.destination.arn}/*"
        Condition = {
          StringEquals = {
            "AWS:SourceArn" = aws_cloudfront_distribution.main.arn
          }
        }
      }
    ]
  })
}

# Lambda Function URL Policy
resource "aws_lambda_function_url" "function_url" {
  function_name = var.lambda_function_name
  authorization_type = "NONE"

  invoke_mode = "BUFFERED"
}

resource "aws_lambda_permission" "allow_cloudfront" {
  statement_id  = "AllowCloudFrontServicePrincipal"
  action        = "lambda:InvokeFunctionUrl"
  function_name = var.lambda_function_name
  principal     = "cloudfront.amazonaws.com"

  source_arn = aws_cloudfront_distribution.main.arn
}