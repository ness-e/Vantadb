$f = "vantadb-python\src\lib.rs"
$content = Get-Content $f -Raw
$orig = $content
$content = $content.Replace("use connectomedb::", "use vantadb::")
$content = $content.Replace("connectomedb::", "vantadb::")
$content = $content.Replace("ConnectomeError", "VantaError")
$content = $content.Replace("ConnectomeDB", "VantaDB")
$content = $content.Replace("NeuronType::STNeuron", "NodeTier::Hot")
$content = $content.Replace("NeuronType::LTNeuron", "NodeTier::Cold")
$content = $content.Replace("NeuronType", "NodeTier")
$content = $content.Replace("neuron_type", "tier")
$content = $content.Replace("CognitiveUnit", "AccessTracker")
$content = $content.Replace("semantic_valence", "importance")
$content = $content.Replace("CONNECTOMEDB_", "VANTADB_")
$content = $content.Replace("CONNECTOME_", "VANTA_")
$content = $content.Replace("connectome_data", "vantadb_data")
$content = $content.Replace("connectome_snapshots", "vantadb_snapshots")
$content = $content.Replace("connectome_query_latency", "vanta_query_latency")
$content = $content.Replace("connectome_oom_circuit", "vanta_oom_circuit")
$content = $content.Replace("connectome_cache_hits", "vanta_cache_hits")
$content = $content.Replace("CXHNSW01", "VNTHNSW1")
$content = $content.Replace("Neuron not found", "Node not found")
$content = $content.Replace("Duplicate neuron", "Duplicate node")
$content = $content.Replace("NexusDB", "VantaDB")
$content = $content.Replace("nexusdb", "vantadb")
$content = $content.Replace(" nexus ", " vantadb ")
$content = $content.Replace("nexusdb_py", "vantadb_py")

if ($content -cne $orig) {
    Set-Content $f -Value $content -NoNewline
    Write-Host "Updated $($f)"
}
