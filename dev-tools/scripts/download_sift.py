import urllib.request
import os
import tarfile

url = "ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz"
file_name = "datasets/sift.tar.gz"
extract_path = "datasets"

print("Downloading SIFT1M (160MB)... This may take a while depending on the FTP server.")
try:
    if not os.path.exists("datasets"):
        os.makedirs("datasets")
        
    urllib.request.urlretrieve(url, file_name)
    print("Download complete. Extracting...")
    
    with tarfile.open(file_name, "r:gz") as tar:
        tar.extractall(path=extract_path)
        
    print("Extraction complete. You can now run the benchmarks!")
except Exception as e:
    print(f"Error downloading or extracting: {e}")
