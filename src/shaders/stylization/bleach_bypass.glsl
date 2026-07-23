    {
        float bl = dot(c, LUMA_BT601);
        vec3 bb = 2.0 * c * vec3(bl);
        vec3 bs = ONE3 - 2.0 * (ONE3 - c) * (ONE3 - vec3(bl));
        c = mix(c, mix(bb, bs, step(0.5, bl)), 0.7);
    }
