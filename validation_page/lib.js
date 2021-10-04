let lib
wasm.wasmmodule.then(r => lib = r)

async function validate_inclusion_proof() {
  try {
    let proof = document.getElementById('inclusionProof').value.trim()
    let node = document.getElementById('node').value.trim()
    let result = await lib.validate_inclusion_proof(proof, node)
    console.log(result);
    addElement(JSON.stringify(result, null, 1))
  } catch (e) {
    addElement(e)
  }
}

function addElement(proof_result) {
  let element = document.getElementById("result");
  element.innerHTML = "<pre>" + "Is valid: " + proof_result + '<br>'
}
