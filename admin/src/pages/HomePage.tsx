import { Link } from 'react-router-dom';
import { Layout } from '../components/Layout';

export function HomePage() {
  return (
    <Layout>
      <div className="max-w-3xl mx-auto text-center">
        <h1 className="text-3xl font-bold mb-6">Welcome to Authava AuthGate Admin</h1>
        <p className="text-lg mb-8">
          Manage your route-based authorization logic with ease. Configure routes, permissions, and access controls.
        </p>

        <div className="bg-white shadow-md rounded-lg p-6 mb-8">
          <h2 className="text-xl font-semibold mb-4">Quick Actions</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Link
              to="/routes"
              className="bg-blue-50 hover:bg-blue-100 border border-blue-200 rounded-lg p-4 flex flex-col items-center"
            >
              <span className="text-lg font-medium text-blue-700 mb-2">View Routes</span>
              <span className="text-gray-600">Browse and manage all routes</span>
            </Link>

            <Link
              to="/routes/new"
              className="bg-green-50 hover:bg-green-100 border border-green-200 rounded-lg p-4 flex flex-col items-center"
            >
              <span className="text-lg font-medium text-green-700 mb-2">Create Route</span>
              <span className="text-gray-600">Add a new route configuration</span>
            </Link>
          </div>
        </div>

        <div className="bg-white shadow-md rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4">About AuthGate</h2>
          <p className="text-gray-600 mb-4">
            AuthGate is a standalone Traefik forwardAuth middleware that authenticates incoming requests using a configurable external session endpoint and authorizes access to protected routes based on roles, permissions, and team-based scopes.
          </p>
          <p className="text-gray-600">
            This admin dashboard allows you to manage your route configurations without editing JSON files directly.
          </p>
        </div>
      </div>
    </Layout>
  );
}