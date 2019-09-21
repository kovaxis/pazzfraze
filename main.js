"use strict";

let pass_gen;

wasm_bindgen("./pkg/wasm_bg.wasm")
    .then(() => {
        const {gen} = wasm_bindgen;
        pass_gen=gen;
    });

window.addEventListener('load', () => {
    const master = document.getElementById("master");
    const master_confirm = document.getElementById("master-confirm");
    const domain = document.getElementById("domain");
    const button = document.getElementById("generate");
    const output = document.getElementById("password");
    button.addEventListener('click', () => {
        if (pass_gen) {
            if (master.value===master_confirm.value) {
                const start = performance.now();
                output.innerText=pass_gen(master.value, domain.value);
                const finish = performance.now();
                console.log("Generated password in "+(finish-start)+"ms");
            }else{
                console.log("Password mismatch");
                alert("Passwords don't match");
            }
        } else {
            console.log("Pazzfraze WASM module not loaded yet");
        }
    });
});
