import { init, update_mouse_position, spawn_entity, print_arena_statistics, tick } from "force-directed-graph";

const ARENA = document.getElementById("arena");

init();


function find_true_position(x, y, element) {
    let parent = element.parent;
    if (parent) {
        let box = getBoundingClientRect();
        let dx = x + box.left;
        let dy = y + box.top;
        if (parent.id === "arena") {
            return [dx, dy];
        } else {
            return find_true_position(dx, dy, parent);
        }
    }
    else {
        return undefined;
    }
}

ARENA.addEventListener("mousemove", event => {
    update_mouse_position(event.clientX, event.clientY);
})

async function loop() {
    let timer = document.getElementById("timer");
    while (true) {
        await new Promise(resolve => setTimeout(resolve, 16));
        tick();
    }
}

loop();
spawn_entity(`<img width="100" src=https://upload.wikimedia.org/wikipedia/commons/thumb/9/9b/DVD_logo.svg/1200px-DVD_logo.svg.png>`, ["greeeeeen"])
const show_details = document.getElementById("status");

show_details.addEventListener("click", event => {
    alert(print_arena_statistics())
});

const info = document.getElementById("yeet");
info.textContent = print_arena_statistics();