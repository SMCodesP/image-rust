import requests
import concurrent.futures
import time
from collections import defaultdict

# Configuração das URLs base para cada domínio e formato
urls = {
    "codes": {
        "webp": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=webp&width=",
        "avif": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=avif&width=",
        "jpeg": "https://d270zblqqzt1pj.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=jpeg&width="
    },
    "aws": {
        "webp": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=webp&width=",
        "avif": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=avif&width=",
        "jpeg": "https://d2w2hn5e1sejqe.cloudfront.net/media/dealerships/17/vehicles/201/ed3c2f00fa69494198fd6122c1f23966.webp?format=jpeg&width="
    }
}

def test_url(domain, formato, width):
    url = f"{urls[domain][formato]}{width}&&&"
    start_time = time.time()
    response = requests.get(url)
    elapsed_time = time.time() - start_time

    req_per_second = 1 / elapsed_time if elapsed_time > 0 else 0
    time_per_request = elapsed_time * 1000  # em milissegundos
    transfer_rate = len(response.content) / 1024 / elapsed_time if elapsed_time > 0 else 0

    return {
        "domain": domain,
        "formato": formato,
        "elapsed_time": elapsed_time,
        "req_per_second": req_per_second,
        "time_per_request": time_per_request,
        "transfer_rate": transfer_rate,
        "image_size": len(response.content)
    }

def compute_summary(results):
    """
    Calcula as médias, o tempo máximo e o tempo mínimo dos resultados fornecidos.
    """
    summary = {
        "avg_elapsed": 0,
        "avg_req_sec": 0,
        "avg_time_req": 0,
        "avg_transfer": 0,
        "avg_image_size": 0,
        "max_elapsed": float('-inf'),
        "min_elapsed": float('inf'),
        "count": 0
    }
    for res in results:
        summary["avg_elapsed"] += res["elapsed_time"]
        summary["avg_req_sec"] += res["req_per_second"]
        summary["avg_time_req"] += res["time_per_request"]
        summary["avg_transfer"] += res["transfer_rate"]
        summary["avg_image_size"] += res["image_size"]
        summary["max_elapsed"] = max(summary["max_elapsed"], res["elapsed_time"])
        summary["min_elapsed"] = min(summary["min_elapsed"], res["elapsed_time"])
        summary["count"] += 1

    if summary["count"] > 0:
        summary["avg_elapsed"] /= summary["count"]
        summary["avg_req_sec"] /= summary["count"]
        summary["avg_time_req"] /= summary["count"]
        summary["avg_transfer"] /= summary["count"]
        summary["avg_image_size"] /= summary["count"]

    return summary

def main():
    widths = range(1000, 1200)  # Range de larguras para teste
    all_results = []

    # Executa os testes em paralelo para cada combinação de domínio, formato e largura
    with concurrent.futures.ThreadPoolExecutor(max_workers=10) as executor:
        futures = []
        for domain in urls:
            for formato in urls[domain]:
                for width in widths:
                    futures.append(executor.submit(test_url, domain, formato, width))
        for future in concurrent.futures.as_completed(futures):
            all_results.append(future.result())

    # Agrupar os resultados por domínio e formato
    results_by_combo = defaultdict(lambda: defaultdict(list))
    for res in all_results:
        results_by_combo[res["domain"]][res["formato"]].append(res)

    # Montar e exibir a comparação final (benchmark) para cada formato em cada domínio
    final_summary = {}
    for domain in results_by_combo:
        final_summary[domain] = {}
        for formato in results_by_combo[domain]:
            summary = compute_summary(results_by_combo[domain][formato])
            final_summary[domain][formato] = summary

    # Exibir os resultados finais
    for domain in sorted(final_summary.keys()):
        print(f"\nBenchmark para {domain}:")
        for formato in sorted(final_summary[domain].keys()):
            summary = final_summary[domain][formato]
            print(f"  Formato: {formato}")
            print(f"    Média do Tempo Total: {summary['avg_elapsed']:.2f}s")
            print(f"    Média de Req/s: {summary['avg_req_sec']:.2f}")
            print(f"    Média do Tempo por Requisição: {summary['avg_time_req']:.2f}ms")
            print(f"    Média da Taxa de Transferência: {summary['avg_transfer']:.2f} kB/s")
            print(f"    Média do Tamanho da Imagem: {summary['avg_image_size']:.2f} Bytes")
            print(f"    Tempo Máximo: {summary['max_elapsed']:.2f}s")
            print(f"    Tempo Mínimo: {summary['min_elapsed']:.2f}s")
    
main()
