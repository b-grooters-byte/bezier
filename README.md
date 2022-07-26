# bezier
Direct2D demo application with Bézier curves. This is a simple example of 
Bézier curves and GUI interaction. There is a GTK4 example and a Windows
Direct2D example.

### Control Points
The control points may be manipulated using the mouse to click in the control
point handle and dragging it around the window. A single control point may be
selected at a time.



![GTK OSX control points](images/Bézier%20OSX.png)

## The Math
The basic equation for a Bézier curve is:

$$
\begin{equation}
P(t) = \sum_{i=0}^n B_i^n(t) * P_i,t \in [0,1]
\end{equation}
$$

where B(t) is the Bernstein polynomial and:

$$
\begin{equation}
B_i^n(t) = \binom{n}{i}t^i(1 - t)^{n-i}, \binom{n}{i} = \frac{n!}{i!(n-i)!}
\end{equation}
$$

The application includes a simple implementation of the curve without any real
attempt at optimization. The only optimization comes in the form of a modification
flag in the Bezier type that is used to recalculate the curve when necessary.

```rust
pub struct Bezier {
    ...
    modified: bool,
}
```

This comes at the cost of having mutable, thread safe ownership of the struct in
the event callbacks for GTK4.

### References
* [Wikipedia](https://en.wikipedia.org/wiki/B%C3%A9zier_curve)
* [Bézier Curve](https://towardsdatascience.com/bézier-curve-bfffdadea212)