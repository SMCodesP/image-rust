#!/bin/bash

urls=(
    "https://d270zblqqzt1pj.cloudfront.net/24.jpeg?width=10&format=webp"
    "https://d270zblqqzt1pj.cloudfront.net/24.jpeg?width=512&format=webp"
    "https://d270zblqqzt1pj.cloudfront.net/24.jpeg?width=1024&format=webp"
)

# Número de requisições e concorrência
num_requests=100
concurrency=10

# Função para extrair os dados relevantes do output do ab
extract_data() {
    local output="$1"
    echo "$output" | awk '
        /Time taken for tests/ { time_taken=$5 }
        /Requests per second/ { requests_per_second=$4 }
        /Time per request/ { time_per_request=$4 }
        /Transfer rate/ { transfer_rate=$3 }
        END {
            printf "%.2f s | %.2f req/s | %.2f ms | %.2f kB/s\n",
                   time_taken, requests_per_second, time_per_request, transfer_rate
        }
    '
}

# Cabeçalho da tabela
printf "%-20s | %-10s | %-12s | %-12s | %-12s\n" "URL" "Tempo Total" "Req/s" "Tempo/Req" "Taxa Transf."
echo "--------------------------------------------------------------------"

# Loop para testar cada URL
for url in "${urls[@]}"; do
    echo "Testando URL: $url" >&2
    output=$(ab -n $num_requests -c $concurrency "$url" 2>/dev/null)
    data=$(extract_data "$output")
    printf "%-20s | %s\n" "$url" "$data"
done
