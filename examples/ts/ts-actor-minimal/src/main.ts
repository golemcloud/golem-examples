var state: number = 0;

export const api = {
    "add": function (value: number) {
        console.log(`Adding ${value} to the counter`);
        state += value;
    },
    "get": function(): number {
        console.log(`Returning the current counter value: ${state}`);
        return state;
    }
}
