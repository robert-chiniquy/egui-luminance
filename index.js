import {
  Scene
} from './pkg';


const s = Scene.new();
var t = 1;

const renderLoop = () => {
  s.tick(t++/10);
  requestAnimationFrame(renderLoop);
};

requestAnimationFrame(renderLoop);