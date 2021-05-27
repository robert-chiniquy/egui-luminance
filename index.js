import {
  App
} from './pkg';


const s = App.new();
var t = 1;

const renderLoop = () => {
  s.tick(t++/50);
  requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);