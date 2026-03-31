import requests
import concurrent.futures
import time
import sys

urls = {
    "codes": {
        "webp": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=webp&width=",
        "avif": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=avif&width=",
        "jpeg": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=jpeg&width="
    },
    "aws": {
        "webp": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=webp&width=",
        "avif": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=avif&width=",
        "jpeg": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/336/4bd33f155717406bb6ae98a39aa47b7b.webp?format=jpeg&width="
    }
}

def test_url(domain, formato, width):
    """
    Realiza a requisição para a URL formada com base no domínio, formato e largura,
    e mede:
      - Tempo total de resposta
      - Requisições por segundo
      - Tempo por requisição (ms)
      - Taxa de transferência (kB/s)
      - Tamanho da imagem (Bytes)
    """
    url = f"{urls[domain][formato]}{width}&"
    start_time = time.time()
    response = requests.get(url)
    elapsed_time = time.time() - start_time

    req_per_second = 1 / elapsed_time if elapsed_time > 0 else 0
    time_per_request = elapsed_time * 1000  # em milissegundos
    transfer_rate = len(response.content) / 1024 / elapsed_time if elapsed_time > 0 else 0

    return {
        "width": width,
        "url": url,
        "elapsed_time": elapsed_time,
        "req_per_second": req_per_second,
        "time_per_request": time_per_request,
        "transfer_rate": transfer_rate,
        "image_size": len(response.content)
    }

def compute_summary(results):
    """
    Calcula as médias, o tempo máximo e o tempo mínimo dos resultados.
    """
    total_time = 0
    total_req = 0
    total_time_req = 0
    total_transfer = 0
    total_image_size = 0
    count = len(results)
    max_time = float('-inf')
    min_time = float('inf')

    for res in results:
        total_time += res["elapsed_time"]
        total_req += res["req_per_second"]
        total_time_req += res["time_per_request"]
        total_transfer += res["transfer_rate"]
        total_image_size += res["image_size"]
        max_time = max(max_time, res["elapsed_time"])
        min_time = min(min_time, res["elapsed_time"])

    return {
        "avg_elapsed": total_time / count,
        "avg_req_sec": total_req / count,
        "avg_time_req": total_time_req / count,
        "avg_transfer": total_transfer / count,
        "avg_image_size": total_image_size / count,
        "max_elapsed": max_time,
        "min_elapsed": min_time
    }

def main():
    # Validação dos argumentos
    if len(sys.argv) != 3:
        print("Uso: python bench.py [platform] [format]")
        print("Exemplo: python bench.py domain1 webp")
        sys.exit(1)

    domain = sys.argv[1]
    formato = sys.argv[2]

    if domain not in urls:
        print(f"Plataforma '{domain}' não encontrada. As opções são: {', '.join(urls.keys())}.")
        sys.exit(1)
    if formato not in urls[domain]:
        print(f"Formato '{formato}' não disponível para '{domain}'. As opções são: {', '.join(urls[domain].keys())}.")
        sys.exit(1)

    # Intervalo de larguras para o teste
    widths = range(1000, 1200)
    results = []

    print(f"Executando benchmark para {domain} no formato {formato}...\n")
    print(f"{'Width':<8} {'URL':<40} {'Tempo Total':<12} {'Req/s':<10} {'Tempo/Req':<12} {'Taxa Transf.':<14} {'Tamanho (Bytes)':<16}")
    print("-" * 120)

    # Executa os testes em paralelo
    with concurrent.futures.ThreadPoolExecutor(max_workers=40) as executor:
        futures = [executor.submit(test_url, domain, formato, width) for width in widths]
        for future in concurrent.futures.as_completed(futures):
            res = future.result()
            results.append(res)
            print(f"{res['width']:<8} {res['url'][:38]:<40} {res['elapsed_time']:.2f}s     "
                  f"{res['req_per_second']:.2f}    {res['time_per_request']:.2f} ms   "
                  f"{res['transfer_rate']:.2f} kB/s   {res['image_size']:<16}")

    # Calcula e exibe o resumo final
    summary = compute_summary(results)
    print("\nResumo Final:")
    print(f"  Média do Tempo Total: {summary['avg_elapsed']:.2f}s")
    print(f"  Média de Req/s: {summary['avg_req_sec']:.2f}")
    print(f"  Média do Tempo por Requisição: {summary['avg_time_req']:.2f}ms")
    print(f"  Média da Taxa de Transferência: {summary['avg_transfer']:.2f} kB/s")
    print(f"  Média do Tamanho da Imagem: {summary['avg_image_size']:.2f} Bytes")
    print(f"  Tempo Máximo: {summary['max_elapsed']:.2f}s")
    print(f"  Tempo Mínimo: {summary['min_elapsed']:.2f}s")

if __name__ == "__main__":
    main()
