import { useState } from 'react'
import { PlusCircle, MoreVertical } from 'lucide-react'
import { cn } from "@/lib/utils"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"

// Add this custom button style
const GreenButton = ({ className, ...props }) => (
  <Button
    className={cn(
      "bg-green-500 hover:bg-green-600 text-white",
      className
    )}
    {...props}
  >
    {...props.children}
  </Button>
)

type Job = {
  id: string
  name: string
  gpuType: string
  imageUrl: string
  status: 'pending' | 'running' | 'completed' | 'failed'
}

export default function Dashboard() {
  const [jobs, setJobs] = useState<Job[]>([
    { id: '1', name: 'Image Classification', gpuType: 'NVIDIA A100', imageUrl: 'https://example.com/image1.jpg', status: 'running' },
    { id: '2', name: 'Object Detection', gpuType: 'NVIDIA V100', imageUrl: 'https://example.com/image2.jpg', status: 'completed' },
    { id: '3', name: 'Semantic Segmentation', gpuType: 'NVIDIA T4', imageUrl: 'https://example.com/image3.jpg', status: 'pending' },
  ])

  const [newJob, setNewJob] = useState({
    name: '',
    gpuType: '',
    imageUrl: '',
  })

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target
    setNewJob(prev => ({ ...prev, [name]: value }))
  }

  const handleSelectChange = (value: string) => {
    setNewJob(prev => ({ ...prev, gpuType: value }))
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const job: Job = {
      id: (jobs.length + 1).toString(),
      ...newJob,
      status: 'pending'
    }
    setJobs(prev => [...prev, job])
    setNewJob({ name: '', gpuType: '', imageUrl: '' })
    console.log('New job created:', job)
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-green-400 to-green-600 p-4">
      <div className="container mx-auto bg-white rounded-lg shadow-lg p-6">
        <h1 className="text-3xl font-bold mb-8">GPU Job Dashboard</h1>

        <Card className="mb-8">
          <CardHeader>
            <CardTitle>Create New Job</CardTitle>
            <CardDescription>Fill in the details to create a new GPU job</CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="name">Job Name</Label>
                  <Input
                    id="name"
                    name="name"
                    placeholder="Enter job name"
                    value={newJob.name}
                    onChange={handleInputChange}
                    required
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="gpu-type">GPU Type</Label>
                  <Select value={newJob.gpuType} onValueChange={handleSelectChange}>
                    <SelectTrigger id="gpu-type">
                      <SelectValue placeholder="Select GPU type" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="NVIDIA A100">NVIDIA A100</SelectItem>
                      <SelectItem value="NVIDIA V100">NVIDIA V100</SelectItem>
                      <SelectItem value="NVIDIA T4">NVIDIA T4</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-2 md:col-span-2">
                  <Label htmlFor="image-url">Image URL</Label>
                  <Input
                    id="image-url"
                    name="imageUrl"
                    placeholder="Enter image URL"
                    value={newJob.imageUrl}
                    onChange={handleInputChange}
                    required
                  />
                </div>
              </div>
              <GreenButton type="submit" className="w-full">
                <PlusCircle className="mr-2 h-4 w-4" /> Create Job
              </GreenButton>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Job List</CardTitle>
            <CardDescription>Manage your GPU jobs</CardDescription>
          </CardHeader>
          <CardContent>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Job Name</TableHead>
                  <TableHead>GPU Type</TableHead>
                  <TableHead>Image URL</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {jobs.map((job) => (
                  <TableRow key={job.id}>
                    <TableCell className="font-medium">{job.name}</TableCell>
                    <TableCell>{job.gpuType}</TableCell>
                    <TableCell className="max-w-xs truncate">{job.imageUrl}</TableCell>
                    <TableCell>
                      <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${
                        job.status === 'completed' ? 'bg-green-100 text-green-800' :
                        job.status === 'running' ? 'bg-green-200 text-green-800' :
                        job.status === 'failed' ? 'bg-red-100 text-red-800' :
                        'bg-yellow-100 text-yellow-800'
                      }`}>
                        {job.status.charAt(0).toUpperCase() + job.status.slice(1)}
                      </span>
                    </TableCell>
                    <TableCell className="text-right">
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <GreenButton variant="ghost" className="h-8 w-8 p-0">
                            <span className="sr-only">Open menu</span>
                            <MoreVertical className="h-4 w-4 text-green-500" />
                          </GreenButton>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuLabel>Actions</DropdownMenuLabel>
                          <DropdownMenuItem>View Details</DropdownMenuItem>
                          <DropdownMenuItem>Pause Job</DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem className="text-red-600">Delete Job</DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}