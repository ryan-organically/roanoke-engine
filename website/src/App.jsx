import Navigation from './components/Navigation'
import Hero from './components/Hero'
import Plot from './components/Plot'
import JohnWhite from './components/JohnWhite'
import Voyage from './components/Voyage'
import Thanksgiving from './components/Thanksgiving'
import VirginiaDare from './components/VirginiaDare'
import Mystery from './components/Mystery'
import Jamestown from './components/Jamestown'
import Pocahontas from './components/Pocahontas'
import Unraveling from './components/Unraveling'
import Themes from './components/Themes'
import Footer from './components/Footer'

function App() {
  return (
    <div className="app">
      <Navigation />
      <main>
        <Hero />
        <Plot />
        <JohnWhite />
        <Voyage />
        <Thanksgiving />
        <VirginiaDare />
        <Mystery />
        <Jamestown />
        <Pocahontas />
        <Unraveling />
        <Themes />
      </main>
      <Footer />
    </div>
  )
}

export default App
