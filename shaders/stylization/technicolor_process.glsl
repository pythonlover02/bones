    {
        vec3 tn = ONE3 - c;
        vec3 tp = vec3(tn.g + tn.b, tn.r + tn.b, tn.r + tn.g) * 0.5;
        c = mix(c, clamp(ONE3 - tp * 0.5, 0.0, 1.0), 0.7);
    }
