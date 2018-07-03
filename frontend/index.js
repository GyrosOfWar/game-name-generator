function getJson() {
  return fetch("names.txt")
    .then(res => res.text());
}

getJson()
  .then(res => {
    const options = {
      order: 6,
      source: res
    };
    const generator = new MarkovChain(options);
    const el = document.getElementById("game-name");
    el.textContent = generator.generate();
  });
