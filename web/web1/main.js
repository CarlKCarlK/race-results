import init, { member_match, sample_members, sample_results } from './pkg/web1.js';

function loadMembersFromFile() {
    var fileInput = document.getElementById('membersFile');
    var file = fileInput.files[0];
    if (file) {
        var reader = new FileReader();
        reader.onload = function (e) {
            document.getElementById('members').value = e.target.result;
        }
        reader.readAsText(file);
    }
}

function loadResultsFromFile() {
    var fileInput = document.getElementById('resultsFile');
    var file = fileInput.files[0];
    if (file) {
        var reader = new FileReader();
        reader.onload = function (e) {
            document.getElementById('results').value = e.target.result;
        }
        reader.readAsText(file);
    }
}


async function callWasmFunction(includeCity) {
    var members = document.getElementById('members').value;
    var results = document.getElementById('results').value;

    document.getElementById('matches').value = "Running ...";
    await sleep(0);

    var wasmOutput = member_match(members, results, includeCity);
    document.getElementById('matches').value = wasmOutput;
}


function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}


window.callWasmFunction = callWasmFunction;
window.loadMembersFromFile = loadMembersFromFile;
window.loadResultsFromFile = loadResultsFromFile;

window.addEventListener('DOMContentLoaded', (event) => {
    init().then(() => {
        document.getElementById('members').value = sample_members();
        document.getElementById('results').value = sample_results();
    });
});