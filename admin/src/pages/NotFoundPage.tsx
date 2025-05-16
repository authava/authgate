import { Link } from 'react-router-dom'
import { Layout } from '../components/Layout'

export function NotFoundPage() {
  return (
    <Layout>
      <div className="max-w-md mx-auto text-center py-12">
        <h1 className="text-6xl font-bold text-white mb-4">404</h1>
        <h2 className="text-2xl font-semibold text-gray-300 mb-6">
          Page Not Found
        </h2>
        <p className="text-gray-400 mb-8">
          The page you are looking for doesn't exist or has been moved.
        </p>
        <Link to="/" className="btn btn-primary">
          Go to Home
        </Link>
      </div>
    </Layout>
  )
}
