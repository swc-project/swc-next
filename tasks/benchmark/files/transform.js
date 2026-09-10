const value = 1;
const object = { value };
const sticky = /transform/y;

class TransformFixture {
    static value = sticky.test("transform") ? object.value : 0;

    static {
        this.value += 1;
    }
}

const getValue = () => TransformFixture.value;

getValue();
