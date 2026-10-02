// @ts-expect-error
const { alpha, beta: renamed, gamma } = readConfig("left")("right");
