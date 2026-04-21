import vantadb_py as vanta

# Inicializar DB en memoria o carpeta local
db = vanta.VantaDB(db_path="./test_vanta_db")
# Insertar un nodo de prueba
db.query("INSERT {id: 'test-1', vector: [0.1] * 384, text: 'Validación VantaDB'}")
# Buscar
res = db.query("SEARCH vector NEAR [0.1] * 384")
print(f"Resultado de búsqueda: {res}")
