import requests
import concurrent.futures
import time

# URL base
base_url = "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/16/vehicles/410/d60c3553bac14b94912ae35602154e12.webp?width="

# Função para medir o tempo de resposta e os dados relevantes da requisição
def test_url(width):
    url = f"{base_url}{width}"
    
    start_time = time.time()
    response = requests.get(url)
    elapsed_time = time.time() - start_time
    
    # A medição é feita com base no tempo de resposta da requisição
    req_per_second = 1 / elapsed_time if elapsed_time > 0 else 0
    time_per_request = elapsed_time * 1000  # em milissegundos
    transfer_rate = len(response.content) / 1024 / elapsed_time if elapsed_time > 0 else 0  # em kB/s
    
    return (url, elapsed_time, req_per_second, time_per_request, transfer_rate)

# Cabeçalho da tabela
print(f"{'URL':<40} {'Tempo Total':<12} {'Req/s':<10} {'Tempo/Req':<12} {'Taxa Transf.':<12}")
print("-" * 80)

# Variáveis para armazenar a soma dos valores para calcular as médias
total_time = 0
total_req_per_second = 0
total_time_per_request = 0
total_transfer_rate = 0
total_urls = 0

# Variáveis para armazenar os tempos máximo e mínimo
max_time = float('-inf')
min_time = float('inf')

# Usando ThreadPoolExecutor para paralelizar as requisições
with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
    # Criar um range de 600 a 800 para os testes de largura
    widths = range(801, 1000)
    results = executor.map(test_url, widths)
    
    # Exibir os resultados de cada URL e acumular as somas
    for result in results:
        url, elapsed_time, req_per_second, time_per_request, transfer_rate = result
        print(f"{url:<40} {elapsed_time:.2f}s {req_per_second:.2f} req/s {time_per_request:.2f} ms {transfer_rate:.2f} kB/s")
        
        # Acumular os valores para a média
        total_time += elapsed_time
        total_req_per_second += req_per_second
        total_time_per_request += time_per_request
        total_transfer_rate += transfer_rate
        total_urls += 1
        
        # Atualizar os tempos máximo e mínimo
        if elapsed_time > max_time:
            max_time = elapsed_time
        if elapsed_time < min_time:
            min_time = elapsed_time

# Calcular as médias
avg_time = total_time / total_urls
avg_req_per_second = total_req_per_second / total_urls
avg_time_per_request = total_time_per_request / total_urls
avg_transfer_rate = total_transfer_rate / total_urls

# Exibir as médias finais
print("\nMédia Final:")
print(f"Tempo Total Médio: {avg_time:.2f}s")
print(f"Req/s Médio: {avg_req_per_second:.2f}")
print(f"Tempo por Requisição Médio: {avg_time_per_request:.2f} ms")
print(f"Taxa de Transferência Média: {avg_transfer_rate:.2f} kB/s")

# Exibir o tempo máximo e mínimo
print("\nTempos Máximo e Mínimo:")
print(f"Tempo Máximo: {max_time:.2f}s")
print(f"Tempo Mínimo: {min_time:.2f}s")
