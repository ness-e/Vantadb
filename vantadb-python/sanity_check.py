import vantadb_py as vanta

db = vanta.Client("./test_vanta_db")
db.insert_node(1, "Validacion VantaDB", [0.1, 0.1, 0.1])
print(db.get_node(1))
print(db.search_vector([0.1, 0.1, 0.1], top_k=1))
db.flush()
db.close()
