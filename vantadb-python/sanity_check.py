import vantadb_py as vanta

db = vanta.VantaDB("./test_vanta_db")
db.insert(1, "Validacion VantaDB", [0.1, 0.1, 0.1])
print(db.get(1))
print(db.search([0.1, 0.1, 0.1], top_k=1))
db.flush()
db.close()
