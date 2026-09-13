import { BrowserRouter, Routes, Route } from 'react-router-dom'
import Layout from './components/Layout'
import HomePage from './pages/HomePage'
import WritePage from './pages/WritePage'
import MyPage from './pages/MyPage'
import PostDetailPage from './pages/PostDetailPage'
import LoginPage from './pages/LoginPage'
import './App.css'

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/login" element={<LoginPage />} />
          <Route path="/write" element={<WritePage />} />
          <Route path="/me" element={<MyPage />} />
          <Route path="/posts/:id" element={<PostDetailPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  )
}

export default App
