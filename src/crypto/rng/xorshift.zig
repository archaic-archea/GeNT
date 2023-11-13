pub fn xorhift(state: usize) usize {
    var x = state;

    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;

    return x;
}
