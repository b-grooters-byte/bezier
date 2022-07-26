# bezier
Direct2D demo application with Bézier curves. This is a simple example of 
Bézier curves and GUI interaction. There is a GTK4 example and a Windows
Direct2D example.

### Control Points
The control points may be manipulated using the mouse to click in the control
point handle and dragging it around the window. A single control point may be
selected at a time.

### GTK4 Drag Operations
The GTK4 implementation demonstrates drag-begin, drag-update, and drag-end 
signals being managed by the application. We created a Bezier render structure
to handle the render state and operations:

```rust
struct BezierRender {
    bezier: Bezier,
    selected: Option<usize>,
}
```
The drag-begin and drag-end operations update the selected control point index and the drag-update updates the curve based on the changes to the selected control point.

We implement the ```Draw``` trait for ```BezierRender``` with ```draw_mut``` implemented. We originally defined the trait with immutable and mutable draw 
functions; however, this may be removed in future iterations.
```rust
impl Draw for BezierRender {
    fn draw_mut(&mut self, ctx: &cairo::Context) {
        ...
    }
}
```


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