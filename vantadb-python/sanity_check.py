import nexusdb_py as nexus

# Inicializar DB en memoria o carpeta local
db = nexus.Vantadb(db_path="./test_nexus_db")
# Insertar un nodo de prueba
db.query("INSERT {id: 'test-1', vector: [0.1] * 384, text: 'Validación Vantadb'}")
# Buscar
res = db.query("SEARCH vector NEAR [0.1] * 384")
print(f"Resultado de búsqueda: {res}")
