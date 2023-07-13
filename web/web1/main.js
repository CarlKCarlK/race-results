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
window.showForm = showForm;

window.addEventListener('DOMContentLoaded', (event) => {
    init().then(() => {
        document.getElementById('members').value = sample_members();
        document.getElementById('results').value = sample_results();
    });
});

function showForm() {
    document.getElementById('valueForm').style.display = 'block';
    document.getElementById('changeButton').style.display = 'none';
}

function hideForm() {  // Add a function to hide the form
    document.getElementById('valueForm').style.display = 'none';
    document.getElementById('changeButton').style.display = 'block';
}

document.getElementById('valueForm').addEventListener('submit', function (event) {
    event.preventDefault();

    let values = {
        prob_member_in_race: parseFloat(document.getElementById('prob_member_in_race').value),
        total_right: parseFloat(document.getElementById('total_right').value),
        total_nickname: parseFloat(document.getElementById('total_nickname').value),
    };

    console.log(values);

    hideForm();  // Hide the form after submitting
});

window.showForm = showForm;
window.hideForm = hideForm;