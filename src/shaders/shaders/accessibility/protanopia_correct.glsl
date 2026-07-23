    {
        vec3 ds = vec3(dot(c, vec3(0.567, 0.433, 0.0)),
                       dot(c, vec3(0.558, 0.442, 0.0)),
                       dot(c, vec3(0.0, 0.242, 0.758)));
        vec3 de = c - ds;
        c = clamp(c + vec3(0.0, de.r * 0.7, de.r * 0.7), 0.0, 1.0);
    }
