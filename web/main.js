"use strict";

let pass_gen;

wasm_bindgen("./wasm_bg.wasm")
    .then(() => {
        const {gen} = wasm_bindgen;
        pass_gen=gen;
    });

window.addEventListener('load', () => {
    const master = document.getElementById("master");
    const domain = document.getElementById("domain");
    const button = document.getElementById("generate");
    const output = document.getElementById("password");
    button.addEventListener('click', () => {
        if (pass_gen) {
            console.log("Generating password");
            output.innerText=pass_gen(master.value, domain.value);
        } else {
            console.log("Pazzfraze WASM module not loaded yet");
        }
    });
});
