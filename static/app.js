bulb_states = {"nightstand": {"rgb": [255, 255, 255], "brightness": 1.0} };

document
    .getElementById('bulb-onoff-nightstand')
    .addEventListener('change', (event) => {
  if (event.currentTarget.checked) {
    bulb_states["nightstand"]["brightness"] = 1.0;
  } else {
    bulb_states["nightstand"]["brightness"] = 0;
  }
  sendBulbStates(bulb_states);
})


function sendBulbStates(bulb_states) {
    fetch("127.0.0.1:8000/api", {
        body: JSON.stringify(bulb_states),
        headers: {
          "Content-Type": "application/json"
        },
        method: "POST"
      })
}

