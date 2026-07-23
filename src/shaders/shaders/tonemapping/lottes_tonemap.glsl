    {
        vec3 lx = max(c, vec3(0.0001));
        vec3 la = pow(lx, vec3(1.6));
        c = la / max(la * pow(lx, vec3(-0.112)) * 0.977 + vec3(0.18), vec3(0.0001));
    }
