import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import { HomePage } from './pages/HomePage'
import { RoutesListPage } from './pages/RoutesListPage'
import { RouteDetailPage } from './pages/RouteDetailPage'
import { RouteCreatePage } from './pages/RouteCreatePage'
import { RouteEditPage } from './pages/RouteEditPage'
import { NotFoundPage } from './pages/NotFoundPage'

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 30000, // 30 seconds
    },
  },
})

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/routes" element={<RoutesListPage />} />
          <Route path="/routes/new" element={<RouteCreatePage />} />
          <Route path="/routes/:id" element={<RouteDetailPage />} />
          <Route path="/routes/:id/edit" element={<RouteEditPage />} />
          <Route path="/404" element={<NotFoundPage />} />
          <Route path="*" element={<Navigate to="/404" replace />} />
        </Routes>
      </BrowserRouter>
    </QueryClientProvider>
  )
}

export default App
