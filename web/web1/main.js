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

    document.getElementById('matches').innerText = "Running ...";
    await sleep(0);

    var wasmOutput = member_match(members, results, includeCity);
    document.getElementById('matches').innerHTML = wasmOutput;
}


function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function convertSpaces(str, n) {
    let chars = str.split('');
    for (let i = 0, count = 0; i < chars.length; i++) {
        if (chars[i] === '\n' || chars[i] === '\u00B6') {
            count++;
            chars[i] = count % n === 0 ? '\n' : '\u00B6';
        }
    }
    return chars.join('');
}

function handleQuantityChange() {
    const n = document.getElementById('quantity').value;
    const textArea = document.getElementById('results');
    textArea.value = convertSpaces(textArea.value, n);
}

window.callWasmFunction = callWasmFunction;
window.loadMembersFromFile = loadMembersFromFile;
window.loadResultsFromFile = loadResultsFromFile;
window.handleQuantityChange = handleQuantityChange;

window.addEventListener('DOMContentLoaded', (event) => {
    init().then(() => {
        document.getElementById('members').value = sample_members();
        document.getElementById('results').value = sample_results();
        document.getElementById('loadingScreen').style.display = 'none';
    });
});

function toggleSettings() {
    var settings = document.getElementById('valueForm');
    if (settings.style.display === 'none') {
        settings.style.display = 'block';
    } else {
        settings.style.display = 'none';
    }
}

window.toggleSettings = toggleSettings;
