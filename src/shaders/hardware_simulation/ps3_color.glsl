    {
        float ps3_lum = dot(c, LUMA_BT601);
        c = c * smoothstep(0.0, 0.06, ps3_lum);
        c = c * vec3(0.98, 1.0, 1.03);
    }
