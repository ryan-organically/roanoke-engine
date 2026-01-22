function Voyage() {
  return (
    <section id="voyage" className="voyage">
      <div className="section-content">
        <h2 className="section-title">The Voyage</h2>
        <span className="event-date">April - July 1587</span>

        <div className="plot-body">
          <p>
            Three ships departed Portsmouth in April: the <em>Lyon</em>, a fly-boat, and a pinnace.
            White had been appointed governor. His pilot was a Portuguese navigator named
            <strong> Simon Fernandes</strong>—a man who would become the colony's first enemy.
          </p>

          <p>
            The voyage was sabotaged from the start. Fernandes abandoned the fly-boat in Portugal.
            He refused to let settlers gather supplies at the Virgin Islands. He promised cattle
            at Hispaniola, then sailed past without stopping. He nearly ran the <em>Lyon</em> aground
            at the Cape of Fear.
          </p>

          <blockquote className="plot-quote">
            When they finally reached Virginia, White planned to check on fifteen men left from
            a previous voyage, then continue to the Chesapeake Bay—the intended site for their
            settlement. Fernandes refused. "It was too late in the summer," he declared.
          </blockquote>

          <p>
            The colonists had no choice. They landed on Roanoke—a place that was never meant
            to be their home. Of the fifteen men left behind, they found only bones.
          </p>

          <div className="fernandes-mystery">
            <p>
              Why did Fernandes strand them? Was he a Spanish agent? A privateer with his own
              agenda? When the fly-boat arrived safely weeks later, Fernandes was <em>irritated</em>—he
              had hoped they would be killed by pirates.
            </p>
            <p className="dramatic-line">
              History does not record his motives. Only that 117 people were abandoned in a place
              they were never meant to stay.
            </p>
          </div>
        </div>
      </div>
    </section>
  )
}

export default Voyage
