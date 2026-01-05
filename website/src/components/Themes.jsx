function Themes() {
  const themes = [
    {
      title: "Ownership vs. Belonging",
      description: "The English sought to own the land. The Powhatan belonged to it. Neither could comprehend the other's relationship with the earth beneath their feet."
    },
    {
      title: "The Cost of Misunderstanding",
      description: "Every conflict between settler and native stemmed not from malice, but from two worldviews so fundamentally different that common ground seemed impossible to find."
    },
    {
      title: "What Survives",
      description: "When a people vanish, what remains? Their words? Their blood, mixed into new generations? Their ghosts, wandering forests that no longer remember their names?"
    },
    {
      title: "The Artist's Eye",
      description: "John White saw what others could not—or would not. His paintings preserved a world that would soon be swept away. But seeing clearly is not the same as being able to change what you see."
    },
    {
      title: "Between Two Worlds",
      description: "Pocahontas, the interpreters, the children of mixed blood—some people became bridges. But bridges get walked on from both sides."
    }
  ]

  return (
    <section id="themes" className="themes">
      <div className="section-content">
        <h2 className="section-title">Themes</h2>
        <div className="themes-grid">
          {themes.map((theme, index) => (
            <div key={index} className="theme-card">
              <h3 className="theme-title">{theme.title}</h3>
              <p className="theme-description">{theme.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

export default Themes
