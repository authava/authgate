import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation } from '@tanstack/react-query'
import { api } from '../api/client'
import { Layout } from '../components/Layout'
import { RouteForm } from '../components/RouteForm'
import { ErrorMessage } from '../components/ErrorMessage'
import type { Route } from '../types'

export function RouteCreatePage() {
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)

  // Create route mutation
  const createRouteMutation = useMutation({
    mutationFn: async (route: Route) => {
      const response = await api.createRoute(route)
      if (response.error) {
        throw new Error(response.error.message)
      }
      return response.data
    },
    onSuccess: (data) => {
      // Navigate to the route detail page
      if (data?.id) {
        navigate(`/routes/${data.id}`)
      } else {
        navigate('/routes')
      }
    },
    onError: (err: Error) => {
      setError(err.message)
      window.scrollTo(0, 0)
    },
  })

  // Handle form submission
  const handleSubmit = (values: Route) => {
    setError(null)
    createRouteMutation.mutate(values)
  }

  return (
    <Layout>
      <div className="max-w-3xl mx-auto">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-bold">Create New Route</h1>
          <button
            className="btn btn-secondary"
            onClick={() => navigate('/routes')}
          >
            Cancel
          </button>
        </div>

        {error && <ErrorMessage message={error} />}

        <div className="bg-white shadow-md rounded-lg p-6">
          <RouteForm
            onSubmit={handleSubmit}
            isLoading={createRouteMutation.isPending}
          />
        </div>
      </div>
    </Layout>
  )
}
