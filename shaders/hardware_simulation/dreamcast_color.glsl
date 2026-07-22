    {
        c = c * 1.15;
        float dc_lum = dot(c, LUMA_BT601);
        c = mix(vec3(dc_lum), c, 1.12);
        c = c + vec3(smoothstep(0.65, 0.95, dc_lum)) * 0.08;
    }
