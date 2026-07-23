float hash21(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float fast_sin(float x) {
    x = x * 0.15915494 + 0.5;
    x = fract(x) * 2.0 - 1.0;
    float a = abs(x);
    return x * (3.5841 - 2.5841 * a) * (1.0 - a);
}

float fast_cos(float x) {
    return fast_sin(x + 1.5707963);
}

const float BAYER4[16] = float[16](
     0.0,  8.0,  2.0, 10.0,
    12.0,  4.0, 14.0,  6.0,
     3.0, 11.0,  1.0,  9.0,
    15.0,  7.0, 13.0,  5.0
);

float bayer_signed(vec2 fc) {
    int ix = int(floor(fc.x)) & 3;
    int iy = int(floor(fc.y)) & 3;
    return (BAYER4[ix + iy * 4] + 0.5) * 0.0625 - 0.5;
}

vec3 ycocg_encode(vec3 x) {
    return vec3(0.25 * x.r + 0.5 * x.g + 0.25 * x.b,
                0.5 * x.r - 0.5 * x.b,
                -0.25 * x.r + 0.5 * x.g - 0.25 * x.b);
}

vec3 ycocg_decode(vec3 y) {
    return vec3(y.x + y.y - y.z, y.x + y.z, y.x - y.y - y.z);
}

vec3 perc_ycc(vec3 x) {
    float y = dot(x, LUMA_BT709);
    return vec3(y, (x.b - y) * 0.5388766, (x.r - y) * 0.6350048);
}

vec3 perc_rgb(vec3 y) {
    float r = y.x + y.z * 1.5748;
    float b = y.x + y.y * 1.8556;
    float g = (y.x - LUMA_BT709.r * r - LUMA_BT709.b * b) * 1.398313;
    return vec3(r, g, b);
}

vec3 hable_map(vec3 x) {
    return ((x * (0.15 * x + 0.05) + 0.004) /
            (x * (0.15 * x + 0.50) + 0.060)) - 0.066666666;
}

vec3 agx_contrast(vec3 x) {
    vec3 x2 = x * x;
    vec3 x4 = x2 * x2;
    return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4
         - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

vec3 uchi_map(vec3 x) {
    const float U_P = 1.0;
    const float U_A = 1.0;
    const float U_M = 0.22;
    const float U_L = 0.4;
    const float U_C = 1.33;
    float u_l0 = (U_P - U_M) * U_L;
    float u_s0 = U_M + u_l0;
    float u_s1 = U_M + U_A * u_l0;
    float u_c2 = (U_A * U_P) / max(U_P - u_s1, 0.0001);
    vec3 u_toe = U_M * pow(max(x, ZERO3) / max(U_M, 0.0001), vec3(U_C));
    vec3 u_lin = vec3(U_M) + U_A * (x - U_M);
    vec3 u_sho = U_P - (U_P - u_s1) * exp(-u_c2 * (x - u_s0));
    return mix(mix(u_toe, u_lin, smoothstep(0.0, U_M, x)),
               u_sho, smoothstep(U_M, u_s0, x));
}

float hud_seg(vec2 p, float mx, float my, float dx, float dy) {
    vec2 d = abs(p - vec2(mx, my)) - vec2(dx, dy);
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float hud_digit(vec2 p, float n) {
    p.y = 1.0 - p.y;
    float s0 = hud_seg(p, 0.5, 0.85, 0.15, 0.0);
    float s1 = hud_seg(p, 0.75, 0.65, 0.0, 0.15);
    float s2 = hud_seg(p, 0.75, 0.25, 0.0, 0.15);
    float s3 = hud_seg(p, 0.5, 0.05, 0.15, 0.0);
    float s4 = hud_seg(p, 0.25, 0.25, 0.0, 0.15);
    float s5 = hud_seg(p, 0.25, 0.65, 0.0, 0.15);
    float s6 = hud_seg(p, 0.5, 0.45, 0.15, 0.0);
    float ge0 = step(0.5, n);
    float ge1 = step(1.5, n);
    float ge2 = step(2.5, n);
    float ge3 = step(3.5, n);
    float ge4 = step(4.5, n);
    float ge5 = step(5.5, n);
    float ge6 = step(6.5, n);
    float ge7 = step(7.5, n);
    float ge8 = step(8.5, n);
    float m0 = 1.0 - ge0;
    float m1 = ge0 - ge1;
    float m2 = ge1 - ge2;
    float m3 = ge2 - ge3;
    float m4 = ge3 - ge4;
    float m5 = ge4 - ge5;
    float m6 = ge5 - ge6;
    float m7 = ge6 - ge7;
    float m8 = ge7 - ge8;
    float m9 = ge8;
    float r0 = m0 * min(s0, min(s1, min(s2, min(s3, min(s4, s5)))));
    float r1 = m1 * min(s1, s2);
    float r2 = m2 * min(s0, min(s1, min(s6, min(s4, s3))));
    float r3 = m3 * min(s0, min(s1, min(s6, min(s2, s3))));
    float r4 = m4 * min(s5, min(s6, min(s1, s2)));
    float r5 = m5 * min(s0, min(s5, min(s6, min(s2, s3))));
    float r6 = m6 * min(s0, min(s5, min(s6, min(s4, min(s3, s2)))));
    float r7 = m7 * min(s0, min(s1, s2));
    float r8 = m8 * min(s0, min(s1, min(s2, min(s3, min(s4, min(s5, s6))))));
    float r9 = m9 * min(s0, min(s1, min(s2, min(s3, min(s5, s6)))));
    return r0 + r1 + r2 + r3 + r4 + r5 + r6 + r7 + r8 + r9;
}
