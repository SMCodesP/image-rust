#!/bin/bash

# URL base
base_url="https://d270zblqqzt1pj.cloudfront.net/media/dealerships/16/vehicles/410/d60c3553bac14b94912ae35602154e12.webp?width="

# Número de requisições (1 por URL)
num_requests=1

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

# Loop para testar a URL com largura variando de 400 até 600
for width in {450..650}; do
    url="${base_url}${width}&format=webp"
    echo "Testando URL: $url" >&2
    output=$(ab -n $num_requests -c 1 "$url" 2>/dev/null)
    data=$(extract_data "$output")
    printf "%-20s | %s\n" "$url" "$data"
done
