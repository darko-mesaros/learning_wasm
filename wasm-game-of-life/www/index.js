import { Universe } from "wasm-game-of-life";

const pre = document.getElementById("game-of-life-canvas");
const universe = Universe.new();

const renderLoop = () => {
  pre.textContent = universe.render();
  universe.tick();

  setTimeout(() => {
    requestAnimationFrame(renderLoop);
  }, 10); // Slows down the animation
};

requestAnimationFrame(renderLoop);
